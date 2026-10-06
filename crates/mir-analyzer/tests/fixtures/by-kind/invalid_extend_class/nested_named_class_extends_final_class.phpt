===description===
A named class declared inside a function body is never collected either
(same gap as the anonymous case) — needs the same check.
===file===
<?php

final class Base {}

function make(): void {
    class Inner extends Base {}
//                      ^^^^ InvalidExtendClass: Class Inner cannot extend final class Base
}
