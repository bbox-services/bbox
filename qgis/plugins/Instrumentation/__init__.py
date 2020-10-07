def serverClassFactory(serverIface):
    from .Instrumentation import Instrumentation
    return Instrumentation(serverIface)
