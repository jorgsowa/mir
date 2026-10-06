===description===
M21: ReflectionClass::hasMethod()/ReflectionParameter::isDefaultValueAvailable()
guard their twin throwing call (getMethod()/getDefaultValue()) on the same
receiver — sibling of the method_exists()/property_exists() free-function
guards, but for Reflection's own instance API. An unguarded call on a
different receiver, or the same receiver without the guard, still flags.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function guardedMethod(string $c, string $n): void {
    $class = new \ReflectionClass($c);
    if ($class->hasMethod($n)) {
        $class->getMethod($n);
    }
}

function unguardedMethod(string $c, string $n): void {
    $class = new \ReflectionClass($c);
    $class->getMethod($n);
//  ^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception ReflectionException is thrown but not declared in @throws
}

function guardedDefault(\ReflectionParameter $p): void {
    if ($p->isDefaultValueAvailable()) {
        $p->getDefaultValue();
    }
}

function unguardedDefault(\ReflectionParameter $p): void {
    $p->getDefaultValue();
//  ^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception ReflectionException is thrown but not declared in @throws
}
