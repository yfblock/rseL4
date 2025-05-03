from os import path
from jinja2 import Environment, PackageLoader, select_autoescape
import sys
import subprocess
from parser import BFTransformer, parser

FILE_DIR = path.dirname(path.realpath(__file__))

env = Environment(
    loader=PackageLoader("templates", ""),
    autoescape=select_autoescape()
)
block_template = env.get_template("block.rs.j2")
tag_template = env.get_template("tag_union.rs.j2")

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


def trans_data(source_file):
    all_data = ""
    tree = parser.parse(getStructure(source_file))
    tree = BFTransformer().transform(tree)
    tagged = {}
    for x in tree.children:
        if x["type"] != "tagged_union":
            continue
        tagged[x["tag_field"]] = x["name"]
    for i in tree.children:
        if i["type"] == "block":
            all_data += block_template.render(i)
        else:
            # TODO: 为 tagged_union 实现类型
            all_data += tag_template.render(i)

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
