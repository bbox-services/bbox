# -*- coding: utf-8 -*-
"""
This script initializes the plugin, making it known to QGIS.
"""


def serverClassFactory(server_iface):
    from . clear_capabilities import ClearCapabilities
    return ClearCapabilities(server_iface)
