===description===
Gap #4: __get return type is used for property-access type inference
===file===
<?php
class Model {
    /** @param string $name */
    public function __get(string $name): string
    {
        return '';
    }
}
function test(Model $model): void {
    $name = $model->name;
    /** @mir-check $name is string */
    echo $name;
    $title = $model->title;
    /** @mir-check $title is string */
    echo $title;
}
===expect===
