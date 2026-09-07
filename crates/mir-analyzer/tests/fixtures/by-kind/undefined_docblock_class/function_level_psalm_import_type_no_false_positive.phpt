===description===
@psalm-import-type on a standalone function imports a class-level alias without false positives
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @psalm-type Coordinate = array{lat: float, lng: float}
 */
class GeoPoint {}

/**
 * @psalm-import-type Coordinate from GeoPoint
 * @param Coordinate $coord
 * @return float
 */
function distanceFromOrigin(array $coord): float {
    return sqrt($coord['lat'] ** 2 + $coord['lng'] ** 2);
}

distanceFromOrigin(['lat' => 3.0, 'lng' => 4.0]);
===expect===
