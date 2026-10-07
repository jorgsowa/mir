===description===
method_exists() keyed by class name (string literal or ::class) suppresses UndefinedMethod on an instance call whose receiver is typed as that class
===file===
<?php
class Widget {}

function literal(Widget $w): void {
    if (method_exists('Widget', 'extra')) {
        $w->extra();
    }
}

function class_const(Widget $w): void {
    if (method_exists(Widget::class, 'extra')) {
        $w->extra();
        $w->extra(...);
    }
}

function negated(Widget $w): void {
    if (!method_exists('Widget', 'extra')) {
        return;
    }
    $w->extra();
}
