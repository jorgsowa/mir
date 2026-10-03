===description===
FALSE POSITIVE reproducer. When a method has @inheritdoc AND its own @return type,
the own type must win — the parent's type must not override it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php
class Animal {}
class Dog extends Animal {}

abstract class AnimalFactory {
    /** @return Animal */
    abstract public function create(): mixed;
}

class DogFactory extends AnimalFactory {
    /**
     * @inheritdoc
     * @return Dog
     */
    public function create(): mixed {
        return new Dog();
    }
}

function test(DogFactory $f): void {
    $d = $f->create();
    /** @mir-check $d is Dog */
    echo get_class($d);
}
===expect===
