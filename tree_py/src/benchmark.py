import s_tree
import random
import timeit

# Generate tree of abritray depth
nNodes = 1000

def generateRandomTree():
    # Initial Nodes
    node = s_tree.Node('root')
    tree = s_tree.Tree(node)
    members = [node]
    n = 1
    while n <= nNodes:
        payload = random.getrandbits(10)
        randomMember = random.choice(members)
        newNode = s_tree.Node(payload)
        tree.add_child(newNode, randomMember)
        members.append(newNode)
        n = n+1
    return members, tree

def find_node_by_id():
    members, tree = generateRandomTree()
    for member in members:
        # Seems to be an oddity with anytree.find() where it returns all the chilren associated with a node, not just the single node
        r = tree.find_by_id(member.id)
        
def move_children():
    members, tree = generateRandomTree()
    for i in range(30):
        randomChild = random.choice(members)
        randomNewParent = random.choice(members)
        childIsAncestor = True
        while (childIsAncestor):
            try:
                tree.move_node(randomChild, randomNewParent)
                childIsAncestor = False
            except :
                randomChild = random.choice(members)
                randomNewParent = random.choice(members)

nTests = 10

t1 = timeit.Timer(generateRandomTree).timeit(number = nTests)
t1avg  = t1/nTests

t2 = timeit.Timer(find_node_by_id).timeit(number = nTests)
t2avg  = t2/nTests

t3 = timeit.Timer(move_children).timeit(number = nTests)
t3avg  = t3/nTests

