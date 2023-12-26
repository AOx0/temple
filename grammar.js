"use strict";
const utils_1 = require("./utils");
module.exports = grammar({
    name: "temple",
    extras: ($) => [$.comment, /\s/],
    inline: ($) => [$.name],
    rules: {
        file: ($) => repeat($.declaration),
        dtype: ($) => choice(seq(optional($.objectt), "{", seq((0, utils_1.commaSep)($.membert), optional(",")), "}"), seq(optional($.arrayt), "[", $.dtype, "]"), $.arrayt, $.boolt, $.numbert, $.stringt, $.anyt),
        objectt: (_) => (0, utils_1.reservedWord)("object"),
        anyt: (_) => (0, utils_1.reservedWord)("any"),
        arrayt: (_) => (0, utils_1.reservedWord)("array"),
        boolt: (_) => (0, utils_1.reservedWord)("bool"),
        numbert: (_) => (0, utils_1.reservedWord)("number"),
        stringt: (_) => (0, utils_1.reservedWord)("string"),
        comment: (_) => token(seq("#", /[^\n]*/)),
        object: ($) => {
            return seq("{", (0, utils_1.commaSep)($.member), "}");
        },
        member: ($) => seq(field("name", $.name), ":", field("value", $._value)),
        membert: ($) => seq(field("name", $.name), ":", field("type", $.dtype)),
        name: ($) => choice($.string, $.identifier),
        identifier: (_) => {
            const identifier_start = /[\$_\p{L}]/;
            const identifier_part = choice(identifier_start, /[0-9]/);
            return token(seq(identifier_start, repeat(identifier_part)));
        },
        declaration: ($) => {
            return seq($.identifier, choice(seq(":", $.dtype, "=", $._value), choice(seq(":", $.dtype), seq("=", $._value))), optional(";"));
        },
        array: ($) => seq("[", (0, utils_1.commaSep)($._value), "]"),
        string: (_) => {
            const double_quote = seq('"', (0, utils_1.repChoice)(seq("\\", choice('"', "\\", "b", "f", "n", "r", "t", "v")), /[^"\\]/), '"');
            const single_quote = seq("'", (0, utils_1.repChoice)(seq("\\", choice("'", "\\", "b", "f", "n", "r", "t", "v")), /[^'\\]/), "'");
            return token(choice(double_quote, single_quote));
        },
        number: (_) => {
            const hex_digit = /[0-9a-fA-F]+/;
            const hex_int = seq("0", /[xX]/, hex_digit);
            const dec_digit = /[0-9]/;
            const exp_part = seq(/[eE]/, optional(/[+-]/), repeat1(dec_digit));
            const int_literal = choice("0", seq(/[1-9]/, repeat(dec_digit)));
            const dec_literal = choice(seq(int_literal, ".", repeat(dec_digit), optional(exp_part)), seq(".", repeat(dec_digit), optional(exp_part)), seq(int_literal, optional(exp_part)));
            return token(seq(/[+-]?/, choice(hex_int, dec_literal, "Infinity", "NaN")));
        },
        null: ($) => "null",
        true: ($) => "true",
        false: ($) => "false",
        _value: ($) => choice($.object, $.array, $.number, $.string, $.null, $.true, $.false),
    },
});
