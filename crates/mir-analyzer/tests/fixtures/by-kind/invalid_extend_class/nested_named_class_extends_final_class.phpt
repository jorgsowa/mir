===description===
A named class declared inside a function body is never collected either
(same gap as the anonymous case) — needs the same check.
===file===
<?php

final class Base {}

function make(): void {
    class Inner extends Base {}
}
===expect===
InvalidExtendClass@6:24-6:28: Class Inner cannot extend final class Base
