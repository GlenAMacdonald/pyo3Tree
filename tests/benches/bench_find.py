from pyo3Tree import TreeMap, NodeMap
from pyo3Tree import Tree, Node
import treelib as tl
# Generate tree of abritray depth
nNodes = 1000
import random
import timeit

nNodes = 1000

def generateRandomTree():
    root = Node()
    tree = Tree(root)
    members = [root]
    n = 1
    while n <= nNodes:
        payload = 'payload'
        randomMember = random.choice(members)
        newNode = Node(payload)
        tree.add(newNode, randomMember)
        members.append(newNode)
        n = n+1
    return members, tree

def generateRandomTreeMap():
    root = NodeMap()
    tree = TreeMap(root)
    members = [root]
    n = 1
    while n <= nNodes:
        payload = random.getrandbits(10)
        randomMember = random.choice(members)
        newNode = NodeMap(payload)
        tree.add(newNode, randomMember)
        members.append(newNode)
        n = n+1
    return members, tree

def generateRandomTreeLib():
    # Initial Nodes
    tlTree = tl.Tree()
    tlTree.create_node('root','root')
    members = ['root']
    n = 1
    while n <= nNodes:
        id = str(random.getrandbits(100))
        payload = random.getrandbits(10)
        randomMemberId = random.choice(members)
        tlTree.create_node(id,id,data = payload, parent = randomMemberId)
        members.append(id)
        n = n+1
    return members, tlTree

def findNodeById():
    members, tree = generateRandomTree()
    for member in members: 
        tree.find_by_id(member.id)

def findNodeByIdMap():
    members, tree = generateRandomTreeMap()
    for member in members: 
        tree.find_by_id(member.id)

def findNodeByIdLib():
    members, tlTree = generateRandomTreeLib()
    for member in members:
        # tlTree.filter_nodes(lambda node: node.identifier == member)
        # NOTE: The following function is a dictionary lookup.
        tlTree.get_node(member)

# def moveNodes():
#     members, tree = generateRandomTree()
#     n = 1
#     moves = 30
#     while n < moves:
#         randomChild = random.choice(members)
#         try:
#             tree.move_node(randomChild, randomNewParent)
#             n = n+1
#         except:
#             ()
        

# def moveNodesMap():
#     members, tree = generateRandomTreeMap()
#     for i in range(30):
#         randomChild = random.choice(members)
#         while ((randomChild in tree.get_ancestors(randomNewParent)) == True) or randomChild.id == randomNewParent.id:
#             randomChild = random.choice(members)
#             randomNewParent = random.choice(members)
#         tree.move_node(randomChild, randomNewParent)

def main():
    nTests = 100
    t1 = timeit.Timer(generateRandomTree).timeit(number = nTests)/nTests
    t2 = timeit.Timer(generateRandomTreeMap).timeit(number = nTests)/nTests
    t3 = timeit.Timer(generateRandomTreeLib).timeit(number = nTests)/nTests
    print("Generating Tree took {:.10f} seconds, on average".format(t1))
    print("Generating TreeMap took {:.10f} seconds, on average".format(t2))
    print("Generating TreeLib took {:.10f} seconds, on average".format(t3))

    nTests = 3
    t1 = timeit.Timer(findNodeById).timeit(number = nTests)/nTests
    t2 = timeit.Timer(findNodeByIdMap).timeit(number = nTests)/nTests
    t3 = timeit.Timer(findNodeByIdLib).timeit(number = nTests)/nTests
    print("Finding every Node by id took {:.10f} seconds, on average".format(t1))
    print("Finding every NodeMap by id took {:.10f} seconds, on average".format(t2))
    print("Finding every TreeLib by id took {:.10f} seconds, on average".format(t3))

if __name__ == "__main__":
    main()