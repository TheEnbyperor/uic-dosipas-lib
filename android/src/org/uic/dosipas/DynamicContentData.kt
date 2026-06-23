package org.uic.dosipas

import android.location.Location

fun DynamicContentData.setCoordinate(location: Location) {
    val coordinate = GeoCoordinate(LatLong(
        location.latitude,
        location.longitude
    ))
    coordinate.setAccuracyFromMeters(location.accuracy.toDouble())
    coordinate.setCoordinateSystem(GeoCoordinateSystem.WGS84)
    setCoordinate(coordinate)
}