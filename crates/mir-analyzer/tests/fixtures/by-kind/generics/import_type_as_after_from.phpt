===description===
`@psalm-import-type X from C as Alias` and `@phpstan-import-type X from C as Alias` bind
the imported alias under the `as` name, like the `X as Alias from C` order.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @phpstan-type Coordinates array{lat: float, lng: float}
 * @psalm-type Coordinates = array{lat: float, lng: float}
 */
final class Geo {}

/**
 * @phpstan-import-type Coordinates from Geo as Point
 * @psalm-import-type Coordinates from Geo as Spot
 * @psalm-import-type Coordinates as Place from Geo
 * @psalm-import-type Coordinates from Geo
 */
final class Map {
    /** @param Point $point */
    public function pin($point): void {}
    /** @param Spot $spot */
    public function mark($spot): void {}
    /** @param Place $place */
    public function visit($place): void {}
    /** @param Coordinates $coordinates */
    public function center($coordinates): void {}
}

$map = new Map();
$map->pin(['lat' => 35.0, 'lng' => 139.0]);
$map->pin(['lat' => 'north', 'lng' => 139.0]);
//        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $point of pin() expects 'array{'lat': float, 'lng': float}', got 'array{'lat': "north", 'lng': 139}'
$map->mark(['lat' => 35.0, 'lng' => 139.0]);
$map->mark(['lat' => 'north', 'lng' => 139.0]);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $spot of mark() expects 'array{'lat': float, 'lng': float}', got 'array{'lat': "north", 'lng': 139}'
$map->visit(['lat' => 'north', 'lng' => 139.0]);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $place of visit() expects 'array{'lat': float, 'lng': float}', got 'array{'lat': "north", 'lng': 139}'
$map->center(['lat' => 'north', 'lng' => 139.0]);
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $coordinates of center() expects 'array{'lat': float, 'lng': float}', got 'array{'lat': "north", 'lng': 139}'
