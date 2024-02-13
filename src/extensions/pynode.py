import functools

__NODE_FUNCTIONS = {}

def node(func=None, format=None):
    # decorator called with params
    if func is None:
        return functools.partial(node, format=format)
    name = func.__name__
    func.__signature_format__ = format
    __NODE_FUNCTIONS[name] = func
    return func
