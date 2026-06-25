===description===
NonStaticSelfCall fires when a non-static method is called via the static:: keyword from a static method.
===file===
<?php
class Widget {
    public function render(): string { return "widget"; }

    public static function renderStatic(): string {
        return static::render();
    }
}
===expect===
NonStaticSelfCall@6:15-6:31: Non-static method Widget::render() cannot be called statically
