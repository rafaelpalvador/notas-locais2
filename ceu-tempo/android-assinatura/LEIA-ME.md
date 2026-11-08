# Chave de assinatura de teste (Android)

`ceu-teste.jks` assina o APK gerado pelo GitHub Actions, para que as atualizações instalem por cima da versão anterior sem precisar de desinstalar.

- Senha: `ceu-teste` · alias: `ceu`
- Serve **só para uso pessoal**: como a chave está no repositório, não deve ser usada para publicar na Play Store. Para publicar, gera uma chave nova e guarda-a nos *Secrets* do GitHub.
