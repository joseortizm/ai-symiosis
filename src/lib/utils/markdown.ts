export function htmlToMarkdown(node: Node): string {
  if (node.nodeType === Node.TEXT_NODE) {
    return node.textContent || ''
  }

  if (node.nodeType !== Node.ELEMENT_NODE) return ''

  const el = node as HTMLElement
  const tag = el.tagName.toLowerCase()
  const children = Array.from(el.childNodes).map(htmlToMarkdown).join('')

  switch (tag) {
    case 'strong':
    case 'b':
      return `**${children}**`
    case 'em':
    case 'i':
      return `*${children}*`
    case 'code':
      return `\`${children}\``
    case 'a':
      return `[${children}](${el.getAttribute('href') || ''})`
    case 'del':
    case 's':
      return `~~${children}~~`
    default:
      return children
  }
}

export function markdownToHtml(text: string): string {
  return text
    .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.*?)\*/g, '<em>$1</em>')
    .replace(/`(.*?)`/g, '<code>$1</code>')
    .replace(/~~(.*?)~~/g, '<del>$1</del>')
    .replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>')
}
