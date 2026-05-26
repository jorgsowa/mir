===description===
Fails with wrong template2
===file===
<?php
class Frozen {}
class Unfrozen {}

/**
 * @template T of Frozen|Unfrozen
 */
class Foo
{
    /**
     * @var T
     */
    private $state;

    /**
     * @param T $state
     */
    public function __construct($state)
    {
        $this->state = $state;
    }

    /**
     * @param string $name
     * @param mixed $val
     * @if-this-is Foo<Unfrozen>
     * @return void
     */
    public function set($name, $val) {}

    /**
     * @return Foo<Frozen>
     */
    public function freeze()
    {
        /** @var Foo<Frozen> */
        $f = clone $this;
        return $f;
    }
}

$f = new Foo(new Unfrozen());
$f->set("asd", 10);
$g = $f->freeze();
$g->set("asd", 20);  // Fails

===expect===
IfThisIsMismatch
===ignore===
TODO
