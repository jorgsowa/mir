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
PossiblyNullMethodCall@5:5-5:40: Cannot call method __toString() on possibly null value
