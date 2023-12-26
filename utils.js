"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.caseInsensitive = exports.reserved = exports.reservedWord = exports.commaSep = exports.optSeq = exports.repSeq = exports.repChoice = exports.optChoice = void 0;
function optChoice(...args) {
    return optional(choice(...args));
}
exports.optChoice = optChoice;
function repChoice(...args) {
    return repeat(choice(...args));
}
exports.repChoice = repChoice;
function repSeq(...args) {
    return repeat(seq(...args));
}
exports.repSeq = repSeq;
function optSeq(...args) {
    return optional(seq(...args));
}
exports.optSeq = optSeq;
function commaSep(elem) {
    return optSeq(elem, repSeq(",", elem), optional(","));
}
exports.commaSep = commaSep;
function reservedWord(word) {
    //return word // when debuging
    return alias(reserved(caseInsensitive(word)), word);
}
exports.reservedWord = reservedWord;
function reserved(regex) {
    return token(prec(1, new RegExp(regex)));
}
exports.reserved = reserved;
function caseInsensitive(word) {
    return word.split('')
        .map(letter => `[${letter}${letter.toUpperCase()}]`)
        .join('');
}
exports.caseInsensitive = caseInsensitive;
