===description===
A class named inside an attribute argument is used, but a wholly unrelated class still gets reported UnusedClass.
===file===
<?php
final class Target {
}

final class Orphan {
}

#[Attribute]
final class Route {
    public function __construct(public string $target) {}
}

#[Route(Target::class)]
final class Consumer {
}

new Consumer();
===expect===
UnusedClass@5:6-6:1: Class Orphan is never referenced
