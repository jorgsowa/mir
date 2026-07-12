===description===
A class named only inside an attribute constructor argument (e.g. `Target::class`) must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class Target {
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
