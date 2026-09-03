===description===
Valid PHP: a list built by appending subtype elements is a `list<Marker>`
(AlphaMarker and BetaMarker both implement Marker). mir keeps the literal array
shape `array{0: AlphaMarker, 1?: BetaMarker}` instead of widening to `list<Marker>`,
so the declared `list<Marker>` return is reported as a mismatch.
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
