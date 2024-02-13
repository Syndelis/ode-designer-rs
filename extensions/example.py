import math
@node
def sen(x):
    return math.sin(x)

@node(format="{}^{}")
def pow(x, y):
    return x ** y

print(__NODE_FUNCTIONS)
