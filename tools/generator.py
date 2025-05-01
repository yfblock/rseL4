from lark import Lark, Transformer, v_args
import subprocess

def getStructure():
    result = subprocess.run(['gcc', '-E', '-P', '-DBF_CANONICAL_RANGE=48', '-x', 'c', '../crates/sel4-types/structures.bf'], capture_output=True, text=True)
    return result.stdout

grammar = open("grammar.lark").read()
parser = Lark(grammar, parser='lalr')

tree = parser.parse(getStructure())

# 可选：转为 Python dict 树
class BFTransformer(Transformer):
    def block(self, items):
        name = str(items[0])
        params = items[1] if isinstance(items[1], list) else []
        fields = items[2 if params else 1:]
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
        return {
            "type": "tag",
            "name": str(items[0]),
            "value": int(items[1])
        }

    def param_list(self, items):
        return list(map(str, items))

    def field(self, items):
        return {"type": "field", "name": str(items[0]), "bits": int(items[1])}

    def field_high(self, items):
        return {"type": "field_high", "name": str(items[0]), "bits": int(items[1])}

    def padding(self, items):
        return {"type": "padding", "bits": int(items[0])}

tree = BFTransformer().transform(tree)
import pprint

pprint.pprint(tree, width=1)
