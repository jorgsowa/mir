===description===
Variable assigned inside is_null() is narrowed in the false branch — no NullableReturnStatement
===file===
<?php
class Model {}

class Repo {
    public function first(): ?Model {
        return null;
    }

    public function firstOrFail(): Model {
        if (!is_null($model = $this->first())) {
            return $model;
        }
        throw new \RuntimeException('Not found');
    }
}
===expect===
