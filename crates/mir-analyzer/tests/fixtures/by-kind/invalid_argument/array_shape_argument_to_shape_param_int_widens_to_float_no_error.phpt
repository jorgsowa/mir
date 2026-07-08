===description===
Shape-to-shape argument passing stays permissive (int literal fits a
float property) — guards against over-tightening the array-compatible
check when fixing the scalar/generic-array-param false negative
===config===
suppress=MissingParamType
===file===
<?php
/** @param array{lat: float, lng: float} $coord */
function distanceFromOrigin(array $coord): float {
    return $coord['lat'] + $coord['lng'];
}
distanceFromOrigin(['lat' => 3, 'lng' => 4]);
===expect===
