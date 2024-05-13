import s_tree

t = s_tree.TTree()

a = s_tree.Thing('a')
b = s_tree.Thing('b')



class TestClass():
        def hello(self):
            return "world"

data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "data": "abc",
    "children": [
        {
            "id": "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8", 
        },
        {
            "id": "b0862e33-81a1-4b26-b152-1f993b5c9349",
            "data": "abc",
            "children": [],
        },
        {
            "id": "d7582511-8d32-47d9-a38a-becceb9b88e7",
            "data": "abc",
            "children": [
                {
                    "id": "9b73a757-da9c-46c0-8ee2-52bd1160ef96",
                    "data": TestClass(),
                    "children": []
                },
                {
                    "id": "d062c7c0-ffff-4c1c-8275-168b8bfe5d39",
                    "data": "abc",
                    "children": []
                },
            ]
        },
    ]
}

import s_tree

t = s_tree.Thing.load(data)

a = s_tree.importa(data)

data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "data": "abc",
    "children": [
        {
            "id": "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8", 
            "data": "abc",
            "children": [],
        },
    ]
}

from s_tree import mTree as Tree
from s_tree import mNode as Node

data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "children": [
        {"id": "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8"},
        {"id": "b0862e33-81a1-4b26-b152-1f993b5c9349"},
        {
            "id": "d7582511-8d32-47d9-a38a-becceb9b88e7",
            "children": [
                {"id": "9b73a757-da9c-46c0-8ee2-52bd1160ef96"},
                {"id": "d062c7c0-ffff-4c1c-8275-168b8bfe5d39"},
            ]
        }
    ]
}

tree = Tree.load(data)
assert isinstance(tree, Tree)


class TestClass():
    def hello(self):
        return "world"

data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "data": TestClass()
}

tree = Tree.load(data)
export_data = tree.export()

assert isinstance(export_data["data"], TestClass)
assert export_data["data"].hello() == "world"


# def test_node_attach():

tree = Tree()
node = Node('a')

tree.add(node)

found = tree.find_by_id(node.id)
assert found is not None
assert found.parent.id

def test_node_attach_deeper():

    tree = Tree(root=Node('b'))
    node = Node('a')

    tree.add(node)

    found = tree.find_by_id(node.id)
    assert found is not None
    assert found.parent.id == tree.root.id


def test_node_attach_deeper_deeper():

tree = Tree(root=Node('root'))
node1 = Node('a')
node2 = Node('b')

tree.add(node1)
tree.add(node2, node1)

found = tree.find_by_id(node2.id)
assert found is not None
assert found.parent.id == node1.id

def test_node_movement():

data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "children": [
        {"id": "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8"},
        {"id": "b0862e33-81a1-4b26-b152-1f993b5c9349"},
        {
            "id": "d7582511-8d32-47d9-a38a-becceb9b88e7",
            "children": [
                {"id": "9b73a757-da9c-46c0-8ee2-52bd1160ef96"},
                {"id": "d062c7c0-ffff-4c1c-8275-168b8bfe5d39"},
            ]
        }
    ]
}

output_data = {
    "id": "343708ec-f679-4345-a7a9-1eb11f974c81",
    "children": [
        {
            "id": "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8",
            "children": [
                {"id": "9b73a757-da9c-46c0-8ee2-52bd1160ef96"},
            ]
        },
        {"id": "b0862e33-81a1-4b26-b152-1f993b5c9349"},
        {
            "id": "d7582511-8d32-47d9-a38a-becceb9b88e7",
            "children": [
                {"id": "d062c7c0-ffff-4c1c-8275-168b8bfe5d39"},
            ]
        }
    ]
}

tree = Tree.load(data)
tgt_node = tree.find_by_id("9b73a757-da9c-46c0-8ee2-52bd1160ef96")
new_parent = tree.find_by_id("dbe14fc0-aeef-4745-a4b0-41c98cbbaea8")

tree.move_node(tgt_node, new_parent)

assert tgt_node.parent.id == "dbe14fc0-aeef-4745-a4b0-41c98cbbaea8"
assert len(tree.find_by_id("d7582511-8d32-47d9-a38a-becceb9b88e7").children) == 1

new_data = tree.export()
assert new_data == output_data