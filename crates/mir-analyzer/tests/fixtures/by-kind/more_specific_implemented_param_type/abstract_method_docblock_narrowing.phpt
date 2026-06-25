===description===
Overriding an abstract method with a more-specific @param docblock must not emit
MethodSignatureMismatch. The native hint stays at the parent type; only the
docblock refines it to a concrete subclass.
===config===
suppress=UnusedParam
===file===
<?php
class Event {}
class ClickEvent extends Event {}
class KeyEvent extends Event {}

abstract class Listener {
    abstract public function handle(Event $event): void;
}

class ClickListener extends Listener {
    /** @param ClickEvent $event */
    public function handle(Event $event): void {}
}

class KeyListener extends Listener {
    /** @param KeyEvent $event */
    public function handle(Event $event): void {}
}
===expect===
