===description===
A class-string<T> property or return read through a bare receiver widens to the template's
bound instead of leaking T as a class name; bound receivers and in-scope reads keep T.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface HasEvents {
    public static function events(): array;
}

final class Subscriber implements HasEvents {
    public static function events(): array { return []; }
}

/** @template T of object */
class Box {
    /** @var class-string<T> */
    public string $name;
    /** @var interface-string<T> */
    public string $contract;

    /**
     * @param class-string<T> $name
     * @param interface-string<T> $contract
     */
    public function __construct(string $name, string $contract) {
        $this->name = $name;
        $this->contract = $contract;
    }

    /** @return class-string<T> */
    public function getName(): string {
        $own = $this->name;
        /** @mir-check $own is class-string<T> */
        return $this->name;
    }
}

/** @template T of HasEvents */
class EventBox {
    /** @var class-string<T> */
    public string $name;

    /** @param class-string<T> $name */
    public function __construct(string $name) {
        $this->name = $name;
    }
}

class Registry {
    /** @var class-string<Subscriber> */
    public string $subscriber = Subscriber::class;
}

function bare(Box $box, EventBox $events, Registry $registry): void {
    $name = $box->name;
    /** @mir-check $name is class-string */
    $contract = $box->contract;
    /** @mir-check $contract is interface-string */
    $returned = $box->getName();
    /** @mir-check $returned is class-string */
    if (is_subclass_of($box->name, HasEvents::class)) {
        print_r($box->name::events());
        print_r($box->getName()::events());
    }

    $bounded = $events->name;
    /** @mir-check $bounded is class-string<HasEvents> */
    print_r($events->name::events());

    $literal = $registry->subscriber;
    /** @mir-check $literal is class-string<Subscriber> */
}

/** @param Box<Subscriber> $box */
function bound(Box $box): void {
    $name = $box->name;
    /** @mir-check $name is class-string<Subscriber> */
    print_r($box->getName()::events());
}

function reflection(ReflectionClass $class): void {
    $name = $class->name;
    /** @mir-check $name is class-string */
    if ($class->isSubclassOf(HasEvents::class)) {
        print_r($class->name::events());
    }
}
