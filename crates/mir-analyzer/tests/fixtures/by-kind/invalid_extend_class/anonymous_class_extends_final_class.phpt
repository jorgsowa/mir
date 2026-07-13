===description===
An anonymous class is never collected into the codebase, so the batch check
(which only sees collected classes) can't catch it — needs its own check.
===file===
<?php

final class Base {}

new class extends Base {};
===expect===
InvalidExtendClass@5:18-5:22: Class <anonymous> cannot extend final class Base
