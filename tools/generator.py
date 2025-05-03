from lark import Lark, Transformer, v_args
from os import path
import sys
import subprocess
import inflection

FILE_DIR = path.dirname(path.realpath(__file__))


def camelize(name) -> str:
    ret = inflection.camelize(name)
    ret = ret.replace("Cnode", "CNode")
    ret = ret.replace("Vspace", "VSpace")
    return ret


def underscore(name) -> str:
    ret = inflection.underscore(name)
    ret = ret.replace("c_node", "cnode")
    ret = ret.replace("v_space", "vspace")
    return ret


def handle_field_name(clzName, fieldName) -> str:
    if fieldName.startswith("cap"):
        fieldName = fieldName.lstrip("cap")
    return fieldName


def derive_str(derives) -> str:
    return "#[derive(%s)]\n" % (",".join(derives))


USIZE_WIDTH = 64

# 参数顺序 (占位大小， 扩展操作（给某些位初始值）)
NEW_FUNC_TEMPLATE = """
    pub const fn empty() -> Self {
        Self([0; %d])%s
    }
"""

# 参数顺序 (field名称，返回类型, idx 顺序，MASK, RightShiftSize)
GET_FUNC_TEMPLATE = """
    pub const fn get_%s(&self) -> %s {
        ((self.0[%d] & 0x%X) >> %d) as _
    }
"""

# 参数顺序
GET_FUNC_BOOL_TEMPLATE = """
    pub const fn get_%s(&self) -> bool {
        ((self.0[%d] & 0x%x) >> %d) == 1
    }
"""

# 参数顺序 (函数可见度, field名称，参数类型, idx 顺序，idx 顺序，MASK，LeftShiftSize)
SET_FUNC_TEMPLATE = """
    %s const fn set_%s(&mut self, value: %s) {
        self.0[%d] = self.0[%d] & !0x%X | ((value as usize) << %d)
    }
"""

# 参数顺序 (函数可见度, field名称, 参数类型，field 名称)
WITH_FUNC_TEMPLATE = """
    %s const fn with_%s(&mut self, value: %s) -> Self {
        self.set_%s(value);
        *self
    }
"""

IMPL_CAP_TRAIT_TEMPLATE = """
impl crate::object::cap::CapTrait for %s {
    fn raw_cap(&self) -> crate::object::cap::RawCap {
        crate::object::cap::RawCap::new(self.0)
    }
}
"""


def getStructure(source_file):
    result = subprocess.run(
        [
            "gcc",
            "-E",
            "-P",
            "-I" + FILE_DIR,
            "-x",
            "c",
            source_file,
        ],
        capture_output=True,
        text=True,
    )
    return result.stdout


lark_template = path.join(FILE_DIR, "grammar.lark")

grammar = open(lark_template).read()
parser = Lark(grammar, parser="lalr")


# 可选：转为 Python dict 树
class BFTransformer(Transformer):
    def block(self, items):
        name = str(items[0])
        params = items[1] if isinstance(items[1], list) else []
        fields = items[2 if params else 1 :]
        return {"type": "block", "name": name, "params": params, "fields": fields}

    def tagged_union(self, items):
        name = str(items[0])
        tag_field = str(items[1])
        tags = items[2:]
        return {
            "type": "tagged_union",
            "name": name,
            "tag_field": tag_field,
            "tags": tags,
        }

    def tag(self, items):
        return {"type": "tag", "name": str(items[0]), "value": int(items[1])}

    def param_list(self, items):
        return list(map(str, items))

    def field(self, items):
        return {"type": "field", "name": str(items[0]), "bits": int(items[1])}

    def field_high(self, items):
        return {"type": "field_high", "name": str(items[0]), "bits": int(items[1])}

    def padding(self, items):
        return {"type": "padding", "bits": int(items[0])}


def trans_data(source_file):
    all_data = ""
    tree = parser.parse(getStructure(source_file))
    tree = BFTransformer().transform(tree)
    tagged = {}
    for x in tree.children:
        if x["type"] != "tagged_union":
            continue
        if x["name"] == "cap":
            x["name"] = "cap_type"
        tagged[x["tag_field"]] = camelize(x["name"])
    for i in tree.children:
        if i["type"] == "block":
            len = sum(field["bits"] for field in i["fields"])
            width = len // USIZE_WIDTH

            top_name = camelize(i["name"])

            declare = derive_str(["Debug", "Clone", "Copy"])
            declare += "#[repr(C)]\n"
            declare += "pub struct %s([usize; %d]); \n" % (top_name, width)
            # 为 Capability 实现 CapTrait
            if "Cap" in top_name:
                declare += IMPL_CAP_TRAIT_TEMPLATE % (top_name)
            declare += "impl %s { " % (top_name)

            is_cap = i["name"].endswith("_cap")
            init_ops = ""
            if is_cap:
                init_ops = ".with_type(CapType::%s as usize)" % (top_name)
            declare += NEW_FUNC_TEMPLATE % (width, init_ops)

            idx = 0
            for field in reversed(i["fields"]):
                field_type = field["type"]
                arg_type = "usize"
                set_with_pub = "pub "

                bit_mask = 0
                shift = idx % USIZE_WIDTH

                # field 正常处理，获取整个值，然后 MASK 特定位就行了
                # TODO: 将仅有一个位的数据更换为 bool
                # TODO: 如果 field 的名称是当前列表中定义的名称需要特殊处理 也就是处理 tagged
                if field_type == "field":
                    bit_mask = (1 << field["bits"]) - 1
                # 处理 field_high 的情况：
                #     padding 16
                #     field_high mdbNext 46
                #     field mdbRevocable 1
                #     field mdbFirstBadged 1
                # 这种情况下， field_high 会其实包含 48 位，但是不携带低两位的信息
                # 所以使用计算后的 MASK (1 << (46 + 2)) - (1 << 2) = 0xFFFF_FFFF_FFFC
                # 真正的 mdbNext 从整个 usize 中获取，然后 &MASK 得到
                elif field_type == "field_high":
                    high = field["bits"] + shift
                    assert high <= 64
                    bit_mask = (1 << high) - (1 << shift)
                    shift = 0
                # padding 不需要产生任何代码，简单跳过就行了
                elif field_type == "padding":
                    idx += field["bits"]
                    continue
                else:
                    raise "无法处理类型 %s" % (field["type"])
                
                if field['bits'] == 1:
                    arg_type = "bool"

                # 判断是不是 Capability，并且根据这个决定是不是公开当前的 field
                # 如果是 Capability 且当前 field 是 type，那么应该不对外暴露设置的接口
                if is_cap and field["name"] == "capType":
                    set_with_pub = ""
                field["name"] = handle_field_name(top_name, field["name"])
                field_name = underscore(field["name"])

                if arg_type == "usize":
                    declare += GET_FUNC_TEMPLATE % (
                        field_name,
                        arg_type,
                        idx / USIZE_WIDTH,
                        bit_mask,
                        shift,
                    )
                elif arg_type == "bool":
                    declare += GET_FUNC_BOOL_TEMPLATE % (
                        field_name,
                        idx / USIZE_WIDTH,
                        bit_mask,
                        shift,
                    )
                else:
                    raise "不能处理的参数类型 %s" % (arg_type)
                declare += SET_FUNC_TEMPLATE % (
                    set_with_pub,
                    field_name,
                    arg_type,
                    idx / USIZE_WIDTH,
                    idx / USIZE_WIDTH,
                    bit_mask,
                    shift,
                )
                declare += WITH_FUNC_TEMPLATE % (set_with_pub, field_name, arg_type, field_name)
                idx += field["bits"]

        else:
            # TODO: 为 tagged_union 实现类型
            top_name = camelize(i["name"])

            declare = "#[repr(usize)]\n"
            declare += derive_str(["Debug", "Clone", "Copy"])
            declare += "pub enum %s { \n" % (top_name)
            for tag in i["tags"]:
                declare += "    %s = %d, \n" % (
                    camelize(tag["name"]),
                    tag["value"],
                )

        declare += "}"
        all_data += declare + "\n\n"

    return all_data


if __name__ == "__main__":
    if len(sys.argv) <= 2:
        print("Please pass the required arguments")
        print("Usage: python3 generator.py [src] [dst]")
        exit(0)
    source_file = sys.argv[1]
    dest_file = sys.argv[2]
    data = trans_data(source_file)
    open(dest_file, "w+").write(data)
