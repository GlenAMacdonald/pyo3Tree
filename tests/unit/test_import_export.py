from pyrontree import Tree

def test_tree_import():

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


def test_tree_export():

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
            },
        ]
    }

    tree = Tree.load(data)
    export_data = tree.export()

    assert data == export_data

def test_object_survives_dehydration():

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