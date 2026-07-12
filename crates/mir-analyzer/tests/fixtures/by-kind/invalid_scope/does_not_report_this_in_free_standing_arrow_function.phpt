===description===
does not report this in free standing arrow function later bound via Closure::bindTo
===file===
<?php
class Container {
    private int $value = 42;
}
$getter = fn (): int => $this->value;
$bound = $getter->bindTo(new Container(), Container::class);
echo $bound();
===expect===
