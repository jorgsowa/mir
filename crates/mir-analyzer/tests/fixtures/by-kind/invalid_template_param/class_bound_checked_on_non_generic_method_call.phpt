===description===
A class-level `@template T of Bound` used to be checked only at `new
Box(...)` construction sites — a receiver typed `Box<NotAnimal>` via a
docblock/param annotation (no constructor call in sight) sailed through
every instance-method and `$var::` static call unchecked, even though the
called method itself declares no template params of its own.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Animal {}
class NotAnimal {}

/** @template T of Animal */
class Box {
    public function get(): void {}
    public static function make(): void {}
}

/** @param Box<NotAnimal> $box */
function test_instance_call_on_bad_receiver_is_flagged($box): void {
    $box->get();
}

/** @param Box<Animal> $box */
function test_instance_call_on_good_receiver_is_silent($box): void {
    $box->get();
}

/** @param Box<NotAnimal> $box */
function test_static_call_on_bad_receiver_is_flagged($box): void {
    $box::make();
}

/** @param Box<Animal> $box */
function test_static_call_on_good_receiver_is_silent($box): void {
    $box::make();
}
===expect===
InvalidTemplateParam@13:4-13:15: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
InvalidTemplateParam@23:4-23:16: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
