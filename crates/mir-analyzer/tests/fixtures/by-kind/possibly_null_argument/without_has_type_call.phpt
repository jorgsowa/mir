===description===
Without has type call
===file===
<?php
$method = new ReflectionMethod(stdClass::class);
$parameters = $method->getParameters();
foreach ($parameters as $parameter) {
    $parameter->getType()->__toString();
}
===expect===
PossiblyNullMethodCall@5:4-5:39: Cannot call method __toString() on possibly null value
