from pyo3Tree import TreeMap, NodeMap
from pyo3Tree import Tree, Node
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

def findNodeById():
    members, tree = generateRandomTree()
    for member in members: 
        tree.find_by_id(member.id)

def findNodeByIdMap():
    members, tree = generateRandomTreeMap()
    for member in members: 
        tree.find_by_id(member.id)

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
    print("Generating Tree took {:.10f} seconds, on average".format(t1))
    print("Generating TreeMap took {:.10f} seconds, on average".format(t2))

    nTests = 3
    t1 = timeit.Timer(findNodeById).timeit(number = nTests)/nTests
    t2 = timeit.Timer(findNodeByIdMap).timeit(number = nTests)/nTests
    print("Finding every Node by id took {:.10f} seconds, on average".format(t1))
    print("Finding every NodeMap by id took {:.10f} seconds, on average".format(t2))

if __name__ == "__main__":
    main()