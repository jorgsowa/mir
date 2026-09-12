===description===
Lists built from subtype elements satisfy their shared base-type return.
===ignore===
===config===
php_version=8.4
===file===
<?php
interface Marker
{
}

final class AlphaMarker implements Marker
{
    public function __construct()
     {
     }
}

final class BetaMarker implements Marker
{
    public function __construct()
     {
     }
}

final class Assembler
{
      /** @return list<Marker> */
    private function createMarkers(bool $withExtra): array
     {
          $markers = [new AlphaMarker()];
        if ($withExtra) {
              $markers[] = new BetaMarker();
          }

        return $markers;
     }
}
===expect===
