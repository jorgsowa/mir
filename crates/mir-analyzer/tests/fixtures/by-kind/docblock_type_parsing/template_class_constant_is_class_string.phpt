===description===
`T::class` in a docblock is the class string of template `T`, not a class named `T::class`.
===file===
<?php
namespace App;

class Base {}
final class Child extends Base {}

final class C {
    /**
     * @template T of object
     * @param T::class $must_extend
     * @return class-string<T>
     */
    public function get(string $must_extend): string {
        echo 'must extend ' . $must_extend;
        return $must_extend;
    }

    public function run(): void {
        $cls = $this->get(Child::class);
        /** @mir-check $cls is class-string<App\Child> */
        echo $cls;
    }
}
