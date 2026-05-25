===description===
exact matching generic parameters still work
===file===
<?php
class Config {
    /** @var Container<string> */
    public $stringContainer;

    /** @var Container<int> */
    public $intContainer;
}

class Container<T> {}

$c = new Config();
$stringContainer = new Container();
$intContainer = new Container();

// Same param types should match
$c->stringContainer = $stringContainer;
$c->intContainer = $intContainer;

// Different concrete params should error (type safety)
// But both bare generics are wildcards so they assign to each other
$c->stringContainer = $intContainer;
===expect===
