
from lark import Lark, Transformer, v_args
from os import path
import inflection

FILE_DIR = path.dirname(path.realpath(__file__))

lark_template = path.join(FILE_DIR, "grammar.lark")

grammar = open(lark_template).read()
parser = Lark(grammar, parser="lalr")


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
    return underscore(fieldName)


class BFTransformer(Transformer):
    index = 0
    def block(self, items):
        name = camelize(str(items[0]))
        params = items[1] if isinstance(items[1], list) else []
        fields = items[2 if params else 1 :]
        # 过滤空的字段
        fields = [item for item in fields if item is not None]
        # 计算真正的偏移后重置 index，因为之前的计算的是从高位向低位的增长
        for item in fields:
            item['name'] = handle_field_name(name, item['name'])
            item['offset'] = self.index - item['offset']
            item['uidx'] = item['offset'] // 64
            item['uoff'] = item['offset'] % 64
            item['bitmask'] = (1 << (item['uoff'] + item['bits'])) - (1 << item['uoff'])
            item['arg'] = 'usize'

            if item['type'] == "field_high":
                item['bits'] += item['uoff']
                item['uoff'] = 0
            if item['bits'] == 1:
                item['arg'] = 'bool'

            del item['offset']
            del item['type']
        size = self.index // 64
        self.index = 0

        return {"type": "block", "size": size, "name": name, "params": params, "fields": fields}

    def tagged_union(self, items):
        name = camelize(str(items[0]))
        if name == "Cap":
            name = "CapType"
        tag_field = str(items[1])
        tags = items[2:]
        return {
            "type": "tagged_union",
            "name": name,
            "tag_field": tag_field,
            "tags": tags,
        }

    def tag(self, items):
        return {"type": "tag", "name": camelize(str(items[0])), "value": int(items[1])}

    def param_list(self, items):
        print(list(map(str, items)))
        return list(map(str, items))

    def field(self, items):
        self.index += int(items[1])
        return {"type": "field", "name": str(items[0]), "bits": int(items[1]), "offset": self.index}


    def field_high(self, items):
        self.index += int(items[1])
        return {"type": "field_high", "name": str(items[0]), "bits": int(items[1]), "offset": self.index}

    def padding(self, items):
        self.index += int(items[0])
