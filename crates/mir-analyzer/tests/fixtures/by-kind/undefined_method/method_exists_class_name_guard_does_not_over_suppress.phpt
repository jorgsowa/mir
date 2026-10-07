===description===
A class-name method_exists() guard suppresses only the guarded method on the guarded class, and only inside the guarded branch
===file===
<?php
class Widget {}
class Gadget {}
class SubWidget extends Widget {}

function other_method(Widget $w): void {
    if (method_exists('Widget', 'extra')) {
        $w->other();
//      ^^^^^^^^^^^ UndefinedMethod: Method Widget::other() does not exist
    }
}

function other_class(Gadget $g): void {
    if (method_exists('Widget', 'extra')) {
        $g->extra();
//      ^^^^^^^^^^^ UndefinedMethod: Method Gadget::extra() does not exist
    }
}

function parent_receiver(Widget $w): void {
    if (method_exists('SubWidget', 'extra')) {
        $w->extra();
//      ^^^^^^^^^^^ UndefinedMethod: Method Widget::extra() does not exist
    }
}

function else_branch(Widget $w): void {
    if (method_exists('Widget', 'extra')) {
        return;
    }
    $w->extra();
//  ^^^^^^^^^^^ UndefinedMethod: Method Widget::extra() does not exist
}
