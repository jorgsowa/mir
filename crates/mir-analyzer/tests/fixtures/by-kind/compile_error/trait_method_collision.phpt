===description===
Two used traits with different bodies for one method collide unless the class declares it or picks one with insteadof; abstract trait methods and the same trait reached twice do not collide.
===file===
<?php
trait Greets {
    public function hello(): string { return 'hi'; }
}

trait Waves {
    public function hello(): string { return 'wave'; }
    public function bye(): string { return 'bye'; }
}

trait NeedsBye {
    abstract public function bye(): string;
}

trait Polite {
    use Greets;
}

class Collides {
    use Greets;
    use Waves;
//      ^^^^^^ CompileError: Trait method Waves::hello has not been applied as Collides::hello, because of collision with Greets::hello [trait-method-collision]
}

class Resolved {
    use Greets, Waves {
        Waves::hello insteadof Greets;
    }
}

class Declares {
    use Greets, Waves;

    public function hello(): string { return 'own'; }
}

class AbstractCounterpart {
    use Waves, NeedsBye;
}

class SameTraitTwice {
    use Polite, Greets;
}
