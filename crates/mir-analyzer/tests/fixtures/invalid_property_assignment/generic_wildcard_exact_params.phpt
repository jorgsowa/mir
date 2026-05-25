===description===
exact matching generic parameters still work
===file===
<?php
/** @template T */
class Container {}

class Config {
    public Container $stringContainer;
    public Container $intContainer;
}

$c = new Config();
$stringContainer = new Container();
$intContainer = new Container();

// Both values are bare generics (wildcards), so assign to either property
$c->stringContainer = $stringContainer;
$c->intContainer = $intContainer;
$c->stringContainer = $intContainer;
===expect===
