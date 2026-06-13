===description===
MixedFunctionCall fires when a variable typed as callable (not mixed) is called
— that should NOT fire; only mixed triggers it.
===file===
<?php
/** @var callable $fn */
$fn = static function(): void {};
$fn();

===expect===
