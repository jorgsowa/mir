===description===
Not all const enums met
===ignore===
TODO
===file===
<?php
class Airport {
    const JFK = "jfk";
    const LHR = "lhr";
    const LGA = "lga";

    /**
     * @param self::* $airport
     */
    public static function getName(string $airport): string {
        return match ($airport) {
            self::JFK => "John F Kennedy Airport",
            self::LHR => "London Heathrow",
        };
    }
}
===expect===
