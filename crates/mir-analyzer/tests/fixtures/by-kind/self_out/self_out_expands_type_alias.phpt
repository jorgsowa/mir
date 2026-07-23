===description===
@psalm-self-out's type never got local type-alias expansion, unlike
@param/@return right next to it -- an alias-named self-out type stayed
an unresolved, unexpandable bare atom instead of retyping the receiver
to the real class it stands for.
===config===
suppress=UnusedParam
===file===
<?php
class Widget {
    public function paint(): void {}
}

class Builder {
    /**
     * @psalm-type WidgetAlias = Widget
     * @psalm-self-out WidgetAlias
     */
    public function build(): void {}
}

$b = new Builder();
$b->build();
$b->paint();
$b->missing();
===expect===
UndefinedMethod@17:0-17:13: Method Widget::missing() does not exist
