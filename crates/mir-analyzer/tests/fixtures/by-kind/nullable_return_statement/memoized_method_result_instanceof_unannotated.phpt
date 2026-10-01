===description===
With memoize_method_call_results, `instanceof` on an unannotated getter's
result narrows the next call of the same getter, directly and through a chain.
===config===
memoize_method_call_results=true
suppress=UnusedVariable,MissingThrowsDocblock
===file===
<?php
interface PropertyType {}
final class PropertyTypeSelect implements PropertyType {
    /** @return list<string> */
    public function getOptions(): array { return []; }
}
class Field {
    public function getPropertyType(): PropertyType { return new PropertyTypeSelect(); }
}
class Form {
    public function getField(): Field { return new Field(); }
}

/** @return list<string> */
function direct(Field $f): array {
    if (!$f->getPropertyType() instanceof PropertyTypeSelect) {
        return [];
    }
    $t = $f->getPropertyType();
    /** @mir-check $t is PropertyTypeSelect */
    return $t->getOptions();
}

/** @return list<string> */
function chained(Form $form): array {
    if (!$form->getField()->getPropertyType() instanceof PropertyTypeSelect) {
        return [];
    }
    return $form->getField()->getPropertyType()->getOptions();
}

/** @return list<string> */
function positiveBranch(Field $f): array {
    if ($f->getPropertyType() instanceof PropertyTypeSelect) {
        return $f->getPropertyType()->getOptions();
    }
    return [];
}
===expect===
