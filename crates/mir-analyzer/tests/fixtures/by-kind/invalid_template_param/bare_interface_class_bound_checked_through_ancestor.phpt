===description===
A bare interface that doesn't redeclare `@template` (`interface
DogContainer extends AnimalContainer {}`) still has its ancestor's
class-level `@template T of Bound` enforced at a method-call site typed
through it — `effective_class_template_params` used to only walk the
single-parent `extends` chain for `ClassLike::Class`, returning `None`
immediately for an interface receiver and silently skipping the bound
check.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'NotAnimal' does not satisfy bound 'Animal'
}

/** @param DogContainer<Animal> $c */
function test_good_receiver_is_silent($c): void {
    $c->get();
}
===expect===
