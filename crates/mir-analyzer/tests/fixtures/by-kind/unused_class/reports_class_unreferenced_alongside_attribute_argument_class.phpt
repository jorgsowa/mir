===description===
A class named inside an attribute argument is used, but a wholly unrelated class still gets reported UnusedClass.
===file===
<?php
final class Target {
}

final class Orphan {
//    ^ +1:1 UnusedClass: Class Orphan is never referenced
}

#[Attribute]
final class Route {
    public function __construct(public string $target) {}
}

#[Route(Target::class)]
final class Consumer {
}

new Consumer();
