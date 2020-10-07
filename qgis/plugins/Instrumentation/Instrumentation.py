import sys
import os

from qgis.server import *
from qgis.core import *
from PyQt5.QtCore import QByteArray


class Instrumentation:

    def __init__(self, serverIface):
        self.serverIface = serverIface

        QgsMessageLog.logMessage("Instrumenation init", 'plugin', Qgis.Info)
        request_filter = InstrumentationRequestFilter(serverIface)
        cache_filter = InstrumentationCacheFilter(serverIface)
        priority = 200
        try:
            QgsLogger.warning("Instrumentation - loading filter")
            serverIface.registerFilter(request_filter, priority)
            serverIface.registerServerCache(cache_filter, priority)
        except Exception as e:
            QgsLogger.warning("Instrumentation - Error loading filter: %s" % e)


class InstrumentationRequestFilter(QgsServerFilter):

    def __init__(self, serverIface):
        super().__init__(serverIface)

    def requestReady(self):
        QgsMessageLog.logMessage("InstrumentationRequestFilter.requestReady")

    def sendResponse(self):
        QgsMessageLog.logMessage("InstrumentationRequestFilter.sendResponse")

    def responseComplete(self):
        QgsMessageLog.logMessage("InstrumentationRequestFilter.responseComplete")


class InstrumentationCacheFilter(QgsServerCacheFilter):

    def __init__(self, serverIface):
        super().__init__(serverIface)

    def getCachedDocument(self, project: QgsProject,
                          request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationCacheFilter.getCachedDocument: %s" %
            project.baseName())
        return QByteArray()

    def getCachedImage(self, project: QgsProject,
                       request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationCacheFilter.getCachedImage")
        return QByteArray()

    def setCachedDocument(self, doc, project: QgsProject,
                          request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationCacheFilter.setCachedDocument")
        return False

    def setCachedImage(self, img: QByteArray, project: QgsProject,
                       request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationCacheFilter.setCachedImage")
        return False

    def deleteCachedDocument(self, project: QgsProject,
                             request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationRequestFilter.deleteCachedDocument")
        return False

    def deleteCachedDocuments(self, project: QgsProject):
        QgsMessageLog.logMessage(
            "InstrumentationRequestFilter.deleteCachedDocuments")
        return False

    def deleteCachedImage(self, project: QgsProject,
                          request: QgsServerRequest, key: str):
        QgsMessageLog.logMessage(
            "InstrumentationRequestFilter.deleteCachedImage")
        return False

    def deleteCachedImages(self, project: QgsProject):
        QgsMessageLog.logMessage(
            "InstrumentationRequestFilter.deleteCachedImages")
        return False
