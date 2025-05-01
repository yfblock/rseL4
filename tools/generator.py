from lark import Lark, Transformer, v_args
import subprocess
import inflection


def camelize(name) -> str:
    return inflection.camelize(name)


def underscore(name) -> str:
    return inflection.underscore(name)


def derive_str(derives) -> str:
    return "#[derive(%s)]\n" % (",".join(derives))


USIZE_WIDTH = 64

NEW_FUNC_TEMPLATE = """
    pub const fn empty() -> Self {
        Self([0; %d])
    }
"""

# 参数顺序 (field名称，idx 顺序，MASK, RightShiftSize)
GET_FUNC_TEMPLATE = """
    pub const fn get_%s(&self) -> usize {
        (self.0[%d] & 0x%X) >> %d
    }
"""

# 参数顺序 (field名称，idx 顺序，idx 顺序，MASK，LeftShiftSize)
SET_FUNC_TEMPLATE = """
    pub const fn set_%s(&mut self, value: usize) {
        self.0[%d] = self.0[%d] & !0x%X | (value << %d)
    }
"""


def getStructure():
    result = subprocess.run(
        [
            "gcc",
            "-E",
            "-P",
            "-DBF_CANONICAL_RANGE=48",
            "-x",
            "c",
            "../crates/sel4-types/structures.bf",
        ],
        capture_output=True,
        text=True,
    )
    return result.stdout


grammar = open("grammar.lark").read()
parser = Lark(grammar, parser="lalr")

tree = parser.parse(getStructure())


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


tree = BFTransformer().transform(tree)
for i in tree.children:
    if i["type"] == "block":
        len = sum(field["bits"] for field in i["fields"])
        width = len // USIZE_WIDTH

        top_name = camelize(i["name"])

        declare = derive_str(["Debug", "Clone", "Copy"])
        declare += "struct %s([usize; %d]); \n" % (top_name, width)
        declare += "impl %s { " % (top_name)
        declare += NEW_FUNC_TEMPLATE % (width)

        idx = 0
        for field in reversed(i["fields"]):
            field_type = field["type"]

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

            field_name = underscore(field["name"])
            declare += GET_FUNC_TEMPLATE % (
                field_name,
                idx / USIZE_WIDTH,
                bit_mask,
                shift,
            )
            declare += SET_FUNC_TEMPLATE % (
                field_name,
                idx / USIZE_WIDTH,
                idx / USIZE_WIDTH,
                bit_mask,
                shift,
            )
            idx += field["bits"]
    else:
        # TODO: 为 tagged_union 实现类型
        top_name = camelize(i["name"])

        declare = "#[repr(usize)]\n"
        declare += derive_str(["Debug", "Clone", "Copy"])
        declare += "enum %s { \n" % (top_name)
        for tag in i["tags"]:
            declare += "    %s = %d, \n" % (
                camelize(tag["name"]),
                tag["value"],
            )

    declare += "}"
    print(declare)
