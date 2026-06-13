===description===
magic properties methods and mixin are available
===config===
suppress=MixedMethodCall,UnusedFunction
===file===
<?php
class User {
    public function name(): string { return ''; }
}

class Delegate {
    public function delegated(): User { return new User(); }
    public string $title = '';
}

/**
 * @property User $owner
 * @psalm-property-read string $label
 * @method User find()
 * @psalm-method string describe()
 * @mixin Delegate
 */
class DynamicModel {}

function test(DynamicModel $model): void {
    $model->owner->name();
    strlen($model->label);
    $model->find()->name();
    strlen($model->describe());
    $model->delegated()->name();
    strlen($model->title);
}
===expect===
MissingConstructor@18:0-18:21: Class DynamicModel has uninitialized properties but no constructor
