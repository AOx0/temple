export function optChoice(...args: RuleOrLiteral[]): Rule {
  return optional(choice(...args));
}

export function repChoice(...args: RuleOrLiteral[]): Rule {
  return repeat(choice(...args));
}

export function repSeq(...args: RuleOrLiteral[]): Rule {
  return repeat(seq(...args));
}

export function optSeq(...args: RuleOrLiteral[]): Rule {
  return optional(seq(...args));
}

export function commaSep(elem: RuleOrLiteral): Rule {
  return optSeq(elem, repSeq(",", elem), optional(","));
}

export function reservedWord(word: string) {
    //return word // when debuging
    return alias(reserved(caseInsensitive(word)), word)
}

export function reserved(regex: string) {
    return token(prec(1, new RegExp(regex)))
}

export function caseInsensitive(word: string) {
    return word.split('')
        .map(letter => `[${letter}${letter.toUpperCase()}]`)
        .join('')
}
