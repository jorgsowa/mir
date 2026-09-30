===description===
Without has type call
===file===
<?php
$method = new ReflectionMethod(stdClass::class);
$parameters = $method->getParameters();
foreach ($parameters as $parameter) {
    $parameter->getType()->__toString();
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method __toString() on possibly null value
}
===expect===
