===file===
<?php
/**
 * @template T
 */
class Box {
    /** @var T */
    private mixed $value;

    /** @param T $value */
    public function __construct(mixed $value) {
        $this->value = $value;
    }

    /** @return T */
    public function get(): mixed {
        return $this->value;
    }
}

class User {
    public function getName(): string { return 'Alice'; }
}

$box = new Box(new User());
$user = $box->get();
$user->getName();
===expect===
