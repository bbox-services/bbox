import sys
import os
import time

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
        self.trace = {}
        self.metrics = {'hit': 0, 'miss': 0, 'count': 0}
        self.projects = {}
        # self.configCache = QgsConfigCache.instance()

    def requestReady(self):
        # Method called when the QgsRequestHandler is ready and populated with
        # parameters, just before entering the main switch for core services.
        self.trace = {'requestReady': time.perf_counter()}
        QgsMessageLog.logMessage("InstrumentationRequestFilter.requestReady")
        request = self.serverInterface().requestHandler()
        # Projects for GetMap queries are cached in QgsConfigCache,
        # with missing public API, so we simulate cache content here.
        # Could also be done by caller outside of FCGI.
        project = request.parameter("MAP") or ''
        if self.projects.get(project) is None:
            self.metrics['miss'] += 1
            self.projects[project] = 1
            # We could load the project here to measure loading time
            self.metrics['count'] += 1
        else:
            self.metrics['hit'] += 1

    def responseComplete(self):
        # Method called when the QgsRequestHandler processing has done and
        # the response is ready, just after the main switch for core services
        # and before final sending response to FCGI stdout.
        self.trace['responseComplete'] = time.perf_counter()
        QgsMessageLog.logMessage("InstrumentationRequestFilter.responseComplete")
        duration = self.trace['responseComplete']-self.trace['requestReady']
        request = self.serverInterface().requestHandler()
        # https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Server-Timing
        # https://developer.mozilla.org/en-US/docs/Tools/Network_Monitor/request_details#timings_tab
        request.setResponseHeader("Server-Timing", "qgis-server;" + str(round(duration*1000000)))
        request.setResponseHeader("X-trace", str(self.trace))
        request.setResponseHeader("X-metrics", str(self.metrics))

    def sendResponse(self):
        QgsMessageLog.logMessage("InstrumentationRequestFilter.sendResponse")


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
