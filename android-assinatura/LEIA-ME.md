# Chave de assinatura de teste (Android)

`notaslocais-teste.jks` assina o APK gerado pelo GitHub Actions, para que as atualizações instalem por cima da versão anterior sem precisar desinstalar.

- Senha: `notaslocais-teste` · alias: `notaslocais`
- Serve **só para uso pessoal**: como a chave está no repositório, ela não deve ser usada para publicar na Play Store. Para publicar, gere uma chave nova e guarde-a em *Secrets* do GitHub.
- Mantenha este repositório **privado**.
