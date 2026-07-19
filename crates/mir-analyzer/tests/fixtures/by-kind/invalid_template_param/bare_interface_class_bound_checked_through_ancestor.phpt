===description===
A bare interface that doesn't redeclare `@template` (`interface
DogContainer extends AnimalContainer {}`) still has its ancestor's
class-level `@template T of Bound` enforced at a method-call site typed
through it — `effective_class_template_params` used to only walk the
single-parent `extends` chain for `ClassLike::Class`, returning `None`
immediately for an interface receiver and silently skipping the bound
check.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
class Animal {}
class NotAnimal {}

/** @template T of Animal */
interface AnimalContainer {
    public function get(): void;
}

interface DogContainer extends AnimalContainer {}

/** @param DogContainer<NotAnimal> $c */
function test_bad_receiver_is_flagged($c): void {
    $c->get();
}

/** @param DogContainer<Animal> $c */
function test_good_receiver_is_silent($c): void {
    $c->get();
}
===expect===
InvalidTemplateParam@14:4-14:13: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
