from qgis.core import Qgis, QgsMessageLog
from qgis.server import QgsServerFilter, QgsConfigCache, QgsCapabilitiesCache
from qgis.PyQt.QtCore import QFileInfo


class ClearCapabilitiesFilter(QgsServerFilter):
    """ QGIS Server ClearCapabilitiesFilter plugin. """

    def __init__(self, server_iface):
        super(ClearCapabilitiesFilter, self).__init__(server_iface)
        self.projects = {}

    def requestReady(self):
        """ Checks the project timestamps and clears cache on update """

        handler = self.serverInterface().requestHandler()
        params = handler.parameterMap()

        if (params.get("SERVICE", "").upper() in ["WMS", "WMTS", "WFS"]
                and params.get("REQUEST", "").upper() in ["GETPROJECTSETTINGS", "GETCAPABILITIES"]
                and params.get("MAP", "")):
            project = params.get("MAP", "")
            fi = QFileInfo(project)

            if fi.exists():
                lm = fi.lastModified()

                if self.projects.get(project, lm) < lm:
                    # QgsConfigCache.instance().removeEntry(project)
                    # cache = QgsCapabilitiesCache()
                    # cache.removeCapabilitiesDocument(project)
                    self.serverInterface().removeConfigCacheEntry(project)

                    QgsMessageLog.logMessage("Cached cleared after update: {} [{}]".format(project, lm.toString()), "ClearCapabilities", Qgis.Warning)

                self.projects[project] = lm


class ClearCapabilities:
    """ Clear Capabilities plugin: this gets loaded by the server at
        start and creates the CLEARCACHE request.
    """

    def __init__(self, server_iface):
        """Register the filter"""
        clear_capabilities = ClearCapabilitiesFilter(server_iface)
        server_iface.registerFilter(clear_capabilities)
